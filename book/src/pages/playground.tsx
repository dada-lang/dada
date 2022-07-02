import "./playground.css";
import React from "react";
import Layout from "@theme/Layout";
import BrowserOnly from "@docusaurus/BrowserOnly";

function PlaygroundBody(): JSX.Element {
  const Ide: typeof import("@site/src/components/Ide").default =
    require("@site/src/components/Ide").default;

  let initialProgramSourceText = `async fn main() {
    print("
        I have forced myself to contradict myself
        in order to avoid conforming to my own taste.
            -- Marcel Duchamp
    ").await
}`;

  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has("code")) {
    initialProgramSourceText = searchParams.get("code") ?? "";
  }

  return (
    <Layout title="Dada playground" description="Dada playground">
      <main>
        <br></br>
        <hr></hr>
        <h1>Dada Playground</h1>

        <Ide mini={false} sourceText={initialProgramSourceText}></Ide>
      </main>
    </Layout>
  );
}

export default function Playground(): JSX.Element {
  return <BrowserOnly>{() => PlaygroundBody()}</BrowserOnly>;
}
