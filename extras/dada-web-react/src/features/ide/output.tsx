import React, { PropsWithChildren } from "react";
import { Graphviz } from "graphviz-react";

import { useAppSelector } from "../../app/hooks";

import { selectCompilerState } from "./ideSlice";

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

function Output() {
  const compilerState = useAppSelector(selectCompilerState);
  const heapBefore = compilerState.heaps[0];
  const heapAfter = compilerState.heaps[1];

  return (
    <>
      <div className="p-2 bg-light">{compilerState.output}</div>
      <div>
        <StateGraph heap={heapBefore} name="State before cursor" />
      </div>
      <div>
        <StateGraph heap={heapAfter} name="State after cursor" />
      </div>
    </>
  );
}

export default Output;
