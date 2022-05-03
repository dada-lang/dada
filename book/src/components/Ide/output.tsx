import React, { PropsWithChildren } from "react";
import { Graphviz } from "graphviz-react";

type StateGraphProps = {
  heap: string;
  name: string;
};

const StateGraph = (props: PropsWithChildren<StateGraphProps>) => {
  if (props.heap === "") return null;

  return (
    <div className='heap-cell'>
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
};

function Output(props: PropsWithChildren<OutputProps>) {
  return (
    <>
      <div className="output p-2 bg-light">{props.output}</div>
      <StateGraph heap={props.heaps[0]} name="State before selected statement" />
      <StateGraph heap={props.heaps[1]} name="State after selected statement" />
    </>
  );
}

export default Output;
