import React, { PropsWithChildren } from "react";
import { Graphviz } from "graphviz-react";

type StateGraphProps = {
  heap: string;
  name: string;
};

const StateGraph = (props: PropsWithChildren<StateGraphProps>) => {
  if (props.heap === "") return null;

  return (
    <>
      <p className="fw-bold">{props.name}</p>
      <Graphviz dot={props.heap} options={{ height: "200px", fit: true }} />
    </>
  );
};

type OutputProps = {
  output: string;
  heaps: [string, string];
};

function Output(props: PropsWithChildren<OutputProps>) {
  return (
    <>
      <div className="output p-2 bg-light">{props.output}</div>
      <div>
        <StateGraph heap={props.heaps[1]} name="Heap state" />
      </div>
    </>
  );
}

export default Output;
