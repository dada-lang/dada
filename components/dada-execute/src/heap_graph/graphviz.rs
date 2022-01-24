use dada_collections::IndexSet;
use dada_id::InternKey;
use dada_parse::prelude::*;

use super::{DataNode, HeapGraph, PermissionNodeLabel, ValueEdge, ValueEdgeTarget};

impl HeapGraph {
    pub fn graphviz(&self, db: &dyn crate::Db, include_temporaries: bool) -> String {
        let mut output = vec![];
        let mut writer = GraphvizWriter {
            db,
            writer: &mut std::io::Cursor::new(&mut output),
            indent: 0,
            include_temporaries,
            node_queue: Default::default(),
            node_set: Default::default(),
            value_edge_list: vec![],
        };
        self.to_graphviz(&mut writer).unwrap();
        String::from_utf8(output).unwrap()
    }

    /*
        digraph G {
        node[shape="rectangle"];

        rankdir = "LR";

        subgraph cluster_stack {
            label=<<b>stack</b>>;
            rank="source";
            stack[shape="none", label=<
                <table border="0">
                <tr><td border="1">foo()</td></tr>
                <tr><td port="0">p</td></tr>
                <tr><td port="1">q</td></tr>
                </table>
            >];
        }

        object[shape="note", label=<
            <table border="0">
            <tr><td border="1">Point</td></tr>
            <tr><td port="0">x</td></tr>
            <tr><td port="1">y</td></tr>
            </table>
        >];

        stack:0 -> object [label="my"];
        stack:1 -> object [label="leased(p)"];
    }
     */

    fn to_graphviz(&self, w: &mut GraphvizWriter<'_>) -> eyre::Result<()> {
        w.indent("digraph {")?;

        w.println(r#"node[shape = "note"];"#)?;

        w.println(r#"rankdir = "LR";"#)?;

        self.print_stack(w)?;

        self.print_heap(w)?;

        let value_edge_list = std::mem::take(&mut w.value_edge_list);
        for value_edge in value_edge_list {
            let label = value_edge.label.as_str();
            w.println(format!(
                r#"{source:?}:{source_port} -> {target:?} [label={label:?}];"#,
                source = value_edge.source,
                source_port = value_edge.source_port,
                target = value_edge.target,
            ))?;
        }

        w.undent("}")?;

        Ok(())
    }

    fn print_stack(&self, w: &mut GraphvizWriter<'_>) -> eyre::Result<()> {
        w.indent("subgraph cluster_stack {")?;
        w.println("label=<<b>stack</b>>")?;
        w.println(r#"rank="source";"#)?;

        w.indent(r#"stack["#)?;
        w.println(r#"shape="none";"#)?;
        w.indent(r#"label=<"#)?;
        w.println(r#"<table border="0">"#)?;
        let mut field_index = 0;
        for stack_frame_node in &self.stack {
            let stack_frame_data = stack_frame_node.data(&self.tables);
            let function_name = stack_frame_data.function.name(w.db).as_str(w.db);
            w.println(format!(r#"<tr><td border="1">{function_name}</td></tr>"#))?;

            let include_temporaries = w.include_temporaries;
            let names = stack_frame_data.variables.iter().map(|v| match v.name {
                Some(word) => Some(word.as_str(w.db).to_string()),
                None if include_temporaries => Some(format!("{:?}", v.id)),
                None => None,
            });

            field_index = self.print_fields(
                w,
                "stack",
                names,
                stack_frame_data.variables.iter().map(|v| &v.value),
                field_index,
            )?;

            if let Some(in_flight_value) = &stack_frame_data.in_flight_value {
                self.print_field(
                    w,
                    in_flight_value,
                    Some(&"(in-flight)".to_string()),
                    "stack",
                    field_index,
                )?;
                field_index += 1;
            }
        }
        w.println(r#"</table>"#)?;
        w.undent(r#">;"#)?;
        w.undent(r#"];"#)?;
        w.undent("}")?;
        Ok(())
    }

    fn print_heap(&self, w: &mut GraphvizWriter<'_>) -> eyre::Result<()> {
        while let Some(edge) = w.node_queue.pop() {
            self.print_heap_node(w, edge)?;
        }
        Ok(())
    }

    fn print_heap_node(&self, w: &mut GraphvizWriter<'_>, edge: ValueEdge) -> eyre::Result<()> {
        let name = w.node_name(&edge);
        w.indent(format!(r#"{name} ["#))?;
        match edge.target {
            ValueEdgeTarget::Object(o) => {
                let data = o.data(&self.tables);
                let fields = data.class.fields(w.db);
                let field_names = fields
                    .iter()
                    .map(|f| Some(f.name(w.db).as_str(w.db).to_string()));
                w.indent(r#"label = <<table border="0">"#)?;
                let class_name = data.class.name(w.db).as_str(w.db);
                w.println(format!(r#"<tr><td border="1">{class_name}</td></tr>"#))?;
                self.print_fields(w, &name, field_names, &data.fields, 0)?;
                w.undent(r#"</table>>"#)?;
            }
            ValueEdgeTarget::Class(c) => {
                let name = c.name(w.db).as_str(w.db);
                w.println(format!(r#"label = <<b>{name}</b>>"#))?;
            }
            ValueEdgeTarget::Function(f) => {
                let name = f.name(w.db).as_str(w.db);
                w.println(format!(r#"label = <<b>{name}()</b>>"#))?;
            }
            ValueEdgeTarget::Data(d) => {
                let data_str = self.data_str(d);
                w.println(format!(r#"label = {data_str:?}"#))?;
            }
        }
        w.undent(r#"];"#)?;
        Ok(())
    }

    fn print_fields<'me>(
        &self,
        w: &mut GraphvizWriter,
        source: &str,
        names: impl IntoIterator<Item = Option<String>>,
        edges: impl IntoIterator<Item = &'me ValueEdge>,
        mut index: usize,
    ) -> eyre::Result<usize> {
        for (edge, name) in edges.into_iter().zip(names) {
            self.print_field(w, edge, name.as_ref(), source, index)?;
            index += 1;
        }
        Ok(index)
    }

    fn print_field(
        &self,
        w: &mut GraphvizWriter,
        edge: &ValueEdge,
        name: Option<&String>,
        source: &str,
        index: usize,
    ) -> Result<(), eyre::Error> {
        let edge: &ValueEdge = edge;
        if let Some(name) = name {
            if let ValueEdgeTarget::Data(d) = edge.target {
                let data_str = self.data_str(d);
                w.println(format!(
                    r#"<tr><td port="{index}">{name}: {data_str}</td></tr>"#,
                ))?;
            } else {
                w.println(format!(r#"<tr><td port="{index}">{name}</td></tr>"#))?;
                let label = edge.permission.data(&self.tables).label;
                w.push_value_edge(source, index, edge, label);
            }
        }
        Ok(())
    }

    fn data_str(&self, d: DataNode) -> String {
        let data_str = format!("{:?}", d.data(&self.tables).debug);
        let data = html_escape::encode_text(&data_str).to_string();
        if data.len() < 40 {
            data
        } else {
            let len = data.len() - 20;
            format!("{}[...]{}", &data[0..20], &data[len..])
        }
    }
}

struct GraphvizWriter<'w> {
    include_temporaries: bool,
    node_queue: Vec<ValueEdge>,
    node_set: IndexSet<ValueEdge>,
    value_edge_list: Vec<GraphvizValueEdge>,
    db: &'w dyn crate::Db,
    writer: &'w mut dyn std::io::Write,
    indent: usize,
}

struct GraphvizValueEdge {
    source: String,
    source_port: usize,
    target: String,
    label: PermissionNodeLabel,
}

impl GraphvizWriter<'_> {
    fn indent(&mut self, s: impl AsRef<str>) -> eyre::Result<()> {
        self.println(s)?;
        self.indent += 2;
        Ok(())
    }

    fn println(&mut self, s: impl AsRef<str>) -> eyre::Result<()> {
        let s = s.as_ref();
        writeln!(
            self.writer,
            "{space:indent$}{s}",
            space = "",
            indent = self.indent,
            s = s
        )?;
        Ok(())
    }

    fn undent(&mut self, s: impl AsRef<str>) -> eyre::Result<()> {
        self.indent -= 2;
        self.println(s)
    }

    fn push_value_edge(
        &mut self,
        source: &str,
        source_port: usize,
        target: &ValueEdge,
        label: PermissionNodeLabel,
    ) {
        let name = self.node_name(target);
        self.value_edge_list.push(GraphvizValueEdge {
            source: source.to_string(),
            source_port,
            target: name,
            label,
        });
    }

    fn node_name(&mut self, edge: &ValueEdge) -> String {
        let (index, new) = self.node_set.insert_full(*edge);
        if new {
            self.node_queue.push(*edge);
        }
        format!("node{index}")
    }
}
