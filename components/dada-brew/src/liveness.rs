//! A simple liveness computation for BIR.

use std::collections::BTreeMap;

use dada_collections::TypedBitSets;
use dada_id::InternKey;
use dada_ir::code::bir::{
    ActionData, Bir, BirData, ControlPoint, ControlPointData, LocalVariable, Place, PlaceData,
    StatementData, TargetPlace, TerminatorData, TerminatorExpr,
};

pub(crate) fn liveness(db: &dyn crate::Db, bir: &Bir) {
    let data = bir.data(db);

    let graph = &data.graph();
    let control_points = data.max_control_point().successor();
    let variables = data.max_local_variable().successor();
    let liveness = Liveness {
        db,
        graph,
        data,
        live_variables: TypedBitSets::new(control_points, variables),
    };
}

struct Liveness<'me> {
    db: &'me dyn crate::Db,

    data: &'me BirData,

    /// Live variables at each point in the graph. This is a bit redundant,
    /// we could just store the bits at terminators and compute the rest,
    /// but I am, well, lazy.
    live_variables: TypedBitSets<ControlPoint, LocalVariable>,

    /// The graph
    graph: &'me BTreeMap<ControlPoint, Vec<ControlPoint>>,
}

impl Liveness<'_> {
    fn compute(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            for (&node, successors) in self.graph {
                for &successor in successors {
                    changed |= self.live_variables.insert_all(node, successor);
                }

                changed |= self.gen_kill(node);
            }
        }
    }

    fn gen_kill(&mut self, node: ControlPoint) -> bool {
        match node.data(&self.data.tables) {
            ControlPointData::Statement(s) => match s.action {
                ActionData::Noop => false,
                ActionData::AssignExpr(_, _) => todo!(),
                ActionData::Clear(_) => todo!(),
                ActionData::BreakpointStart(_, _) => todo!(),
                ActionData::BreakpointEnd(_, _, _, _) => todo!(),
            },
            ControlPointData::Terminator(t) => match t {
                TerminatorData::Assign(target_place, expr, _) => match expr {
                    TerminatorExpr::Await(place) => self.gen(node, std::slice::from_ref(place)),
                    TerminatorExpr::Call {
                        function,
                        arguments,
                        labels,
                    } => self.gen(node, std::slice::from_ref(function)) | self.gen(node, arguments),
                },
                TerminatorData::Return(_) => todo!(),

                // No reads or writes of anything:
                TerminatorData::Goto(_)
                | TerminatorData::If(_, _, _)
                | TerminatorData::StartAtomic(_)
                | TerminatorData::EndAtomic(_)
                | TerminatorData::Error
                | TerminatorData::Panic => false,
            },
        }
    }

    fn gen(&mut self, node: ControlPoint, places: &[Place]) -> bool {
        let mut changed = false;
        for place in places {
            if let Some(var) = self.var(place) {
                changed |= self.live_variables.insert(node, var);
            }
        }
        changed
    }

    fn kill(&mut self, node: ControlPoint, place: TargetPlace) -> bool {
        let mut changed = false;
        for place in places {
            if let Some(var) = self.var(place) {
                changed |= self.live_variables.insert(node, var);
            }
        }
        changed
    }

    fn var(&self, place: &Place) -> Option<LocalVariable> {
        match place.data(&self.data.tables) {
            PlaceData::LocalVariable(lv) => Some(*lv),
            PlaceData::Function(_) => None,
            PlaceData::Class(_) => None,
            PlaceData::Intrinsic(_) => None,
            PlaceData::Dot(place, _) => self.var(place),
        }
    }
}
