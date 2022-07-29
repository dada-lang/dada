import React, { PropsWithChildren } from "react";
import { Graphviz } from "graphviz-react";
import { OutputMode } from ".";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import { Stack } from "react-bootstrap";

type StateGraphProps = {
  heap: string;
  name: string;
};

const StateGraph = (props: PropsWithChildren<StateGraphProps>) => {
  if (props.heap === "") return <div className="heap-cell"></div>;

  return (
    <div className="heap-cell">
      <h3>{props.name}</h3>
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
  mini: boolean;
};

function Output(props: PropsWithChildren<OutputProps>) {
  if (props.mini) {
    const outputTemplate = (
      <div
        className="dada-output p-2 bg-light"
        dangerouslySetInnerHTML={{ __html: props.output }}
      ></div>
    );

    switch (props.mode) {
      case OutputMode.EXECUTE:
        return outputTemplate;
      case OutputMode.DEBUG:
        return (
          <Row>
            <Col>{outputTemplate}</Col>
            <Col>
              <StateGraph heap={props.heaps[1]} name="State at cursor" />
            </Col>
          </Row>
        );
      case OutputMode.NONE:
        return <div className="dada-ir-output p-2 bg-light"></div>;
      default:
        return outputTemplate;
    }
  } else {
    const executeTemplate = (
      <>
        <div
          className="dada-output p-2 bg-light"
          dangerouslySetInnerHTML={{ __html: props.output }}
        ></div>
      </>
    );

    const debugTemplate = (
      <>
        <div
          className="dada-output p-2 bg-light"
          dangerouslySetInnerHTML={{ __html: props.output }}
        ></div>
        <StateGraph
          heap={props.heaps[1]}
          name="State after selected statement"
        />
      </>
    );

    const irTemplate = (
      <>
        <div
          className="dada-ir-output p-2 bg-light"
          dangerouslySetInnerHTML={{ __html: props.output }}
        ></div>
      </>
    );

    switch (props.mode) {
      case OutputMode.EXECUTE:
        return executeTemplate;
      case OutputMode.DEBUG:
        return debugTemplate;
      case OutputMode.NONE:
        return <div className="dada-ir-output p-2 bg-light"></div>;
      default:
        return irTemplate;
    }
  }
}

export default Output;
