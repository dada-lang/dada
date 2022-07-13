import React, { PropsWithChildren } from "react";
import { Graphviz } from "graphviz-react";
import { OutputMode } from ".";

type StateGraphProps = {
  heap: string;
  name: string;
};

const StateGraph = (props: PropsWithChildren<StateGraphProps>) => {
  if (props.heap === "") return null;

  return (
    <div className="heap-cell">
      <h2>{props.name}</h2>
      <div className="heap">
        <Graphviz dot={props.heap} options={{ height: "200px", fit: true }} />
      </div>
    </div>
  );
};

type OutputProps = {
  output: string;
  heaps: [string, string];
  mode: OutputMode;
};

function Output(props: PropsWithChildren<OutputProps>) {
  const executeTemplate = (
    <>
      <div
        className="execute-output p-2 bg-light"
        dangerouslySetInnerHTML={{ __html: props.output }}
      ></div>
      <StateGraph
        heap={props.heaps[0]}
        name="State before selected statement"
      />
      <StateGraph heap={props.heaps[1]} name="State after selected statement" />
    </>
  );

  const irTemplate = (
    <>
      <div
        className="ir-output p-2 bg-light"
        dangerouslySetInnerHTML={{ __html: props.output }}
      ></div>
    </>
  );

  return props.mode === OutputMode.EXECUTE ? executeTemplate : irTemplate;
}

export default Output;
