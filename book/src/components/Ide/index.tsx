import React from 'react';
import { useEffect, useState } from "react";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import {default as AnsiUp} from 'ansi_up';

import dadaWeb, { compiler } from "dada-web";
import type { DadaCompiler, InitOutput } from "dada-web";

import Editor from "./editor";
import Output from "./output";

/**
 * The Queue serializes a number of things to execute one after another.
 * This is used because the Dada compiler for a given Ide cannot be
 * accessed many times concurrently.
 */
class Queue {
    active: number;
    queue: Array<() => Promise<void>>;

    constructor() {
        this.active = 0;
        this.queue = [];
    }

    // Submit a workFunction to the queue -- when called, this
    // should return a promise. It will be called once the
    // active worker has gotten around to it.
    submit(workFunction: () => Promise<void>) {
        this.queue.push(workFunction);

        if (!this.active) {
            this.active = 1;
            this.doWork();
        }
    }

    async doWork() {
        while (this.queue.length != 0) {
            let workFunction = this.queue.shift();
            let promise = workFunction();
            await promise;
        }
        this.active = 0;
    }
}


/**
 * Wrapper on the DadaCompiler to have a stable reference.
 */
class DCW {
    dada: DadaCompiler;
    constructor() {
        this.dada = compiler();
    }

    setSourceText(text: string) {
        this.dada = this.dada.with_source_text(text);
    }

    setBreakpoint(row: number, column: number) {
        if (row && column) this.dada = this.dada.with_breakpoint(row, column);
        else this.dada = this.dada.without_breakpoint();
    }

    async execute() {
        this.dada = await this.dada.execute();
    }

    get output() {
        return this.dada.output;
    }

    get diagnostics() {
        return this.dada.diagnostics;
    }

    get num_diagnostics() {
        return this.dada.num_diagnostics;
    }

    diagnostic(index: number) {
        return this.dada.diagnostic(index);
    }

    get num_breakpoint_ranges() {
        return this.dada.num_breakpoint_ranges;
    }

    breakpoint_range(index: number) {
        return this.dada.breakpoint_range(index);
    }

    get heaps() {
        return [this.dada.heap_before, this.dada.heap_after];
    }
}

export type Cursor = { row: number; column: number };

function Ide(props: { mini: boolean, sourceText: string }) {
    const queue = new Queue();
    const [_module, setModule] = useState<InitOutput | null>(null);
    const [dada, setDada] = useState<DCW | null>(null);

    // First pass: we have to initialize the webassembly and "DCW"
    // instance.
    useEffect(() => {
        async function initModule() {
            // Load the web assembly module
            const c = await dadaWeb();
            setModule(c);
            setDada(new DCW());
        }
        initModule();
    }, []);


    // Second pass: now that `dada != null`, we can do the rest.
    const [cursor, setCursor] = useState<Cursor>({ row: 0, column: 0 });
    const [source, setSource] = useState<string>(props.sourceText);
    const [status, setStatus] = useState<string>("");
    const [diagnostics, setDiagnostics] = useState<string>("");
    const [output, setOutput] = useState<string>("");
    const [heaps, setHeaps] = useState<[string, string]>(["", ""]);
    useEffect(() => {
        queue.submit(async function () {
            if (!dada) return;
            dada.setSourceText(source);
            dada.setBreakpoint(cursor.row, cursor.column);
            await dada.execute();
            const html = new AnsiUp().ansi_to_html(dada.output);
            setOutput(html);
            setHeaps([dada.heaps[0], dada.heaps[1]]);
            setDiagnostics(dada.diagnostics);
        });
    }, [cursor, dada, source]);

    useEffect(() => {
        setSource(props.sourceText);
    }, [props]);

    if (props.mini) {


        return (
            <Row>
                <Col>
                    <Editor source={source} onCursorChange={setCursor} onSourceChange={setSource} />
                </Col>
                <Col>
                    <Output output={output} heaps={heaps} />
                </Col>
            </Row>
        );
    } else {
        return (
            <div className='ide'>
                <div className="editor-cell">
                    <input type="button" value="share" onClick={() => copyClipboardUrl(source, setStatus)} />
                    <span>{status}</span>
                    <Editor source={source} onCursorChange={setCursor} onSourceChange={setSource} />
                </div>
                <div className="output-cell">
                    <Output output={output} heaps={heaps} />
                </div>
            </div>
        );
    }
}

export default Ide;

async function copyClipboardUrl(source, setStatus) {
    // get URL of the playground, and clear existing parameters
    var playgroundUrl = new URL(document.location.href);
    playgroundUrl.search = "?"; // clear existing parameters

    // set the ?code=xxx parameter
    playgroundUrl.searchParams.set("code", source);

    // minify
    let minifiedUrl = await minify(playgroundUrl);
    await navigator.clipboard.writeText(minifiedUrl.href);

    setStatus("url copied to clipboard");
}

// Use the is.gd service to minify a URL.
// If the request fails, returns the unminified URL.
async function minify(url) {
    // Use the is.gd 
    // ?format=simple&url=www.example.com

    let isGdUrl = new URL("https://is.gd/create.php");
    isGdUrl.searchParams.set("format", "simple");
    isGdUrl.searchParams.set("url", url.href);

    try {
        let response = await fetch(isGdUrl);
        let text = await response.text();
        return new URL(text);
    } catch (e) {
        return url;
    }
}
