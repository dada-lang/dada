import React from 'react';
import Layout from '@theme/Layout';
import Ide from '@site/src/components/Ide';

export default function Playground(): JSX.Element {
    let initialProgramSourceText = `
async fn main() {
    print("
        I have forced myself to contradict myself
        in order to avoid conforming to my own taste.
            -- Marcel Duchamp
    ").await
}
`;

    return (
        <Layout
            title="Dada playground"
            description="Dada playground">

            <main>
                <br></br>
                <hr></hr>
                <h1>Dada Playground</h1>

                <input id="shareButton" type="button" value="share" />
                <span id="statusSpan"></span>
                <Ide sourceText={initialProgramSourceText}></Ide>
            </main>
        </Layout>
    );
}

async function copyClipboardUrl(editor) {
    // get URL of the playground, and clear existing parameters
    var playgroundUrl = new URL(document.location.href);
    playgroundUrl.search = "?"; // clear existing parameters

    // set the ?code=xxx parameter
    let code = editor.getValue();
    playgroundUrl.searchParams.set("code", code);

    // minify
    let minifiedUrl = await minify(playgroundUrl);
    await navigator.clipboard.writeText(minifiedUrl.href);

    setStatusMessage("url copied to clipboard");
}