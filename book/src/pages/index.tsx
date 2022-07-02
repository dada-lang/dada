import React from "react";
import clsx from "clsx";
import Layout from "@theme/Layout";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import styles from "./index.module.css";
// import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <Row>
          <Col>
            <img
              src="https://raw.githubusercontent.com/dada-lang/dada-artwork/main/dada.svg"
              width="600"
            ></img>
          </Col>
          <Col>
            <blockquote className="rectangle-speech-border">
              <h1 className="dada-left-justify">
                Welcome to <b>Dada</b>, an experimental new programming
                language!
              </h1>

              <h3 className="dada-left-justify">
                <a href="/docs/dyn_tutorial">Care to try our live tutorial?</a>
              </h3>
              <h3 className="dada-left-justify">
                <a href="/docs/about">
                  Wondering what the heck Dada is all about?
                </a>
              </h3>
              <h3 className="dada-left-justify">
                <a href="/playground">Jump to the web playground?</a>
              </h3>
            </blockquote>
          </Col>
        </Row>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title}`}
      description="Dada is a WIP programming language"
    >
      <HomepageHeader />

      <main></main>
    </Layout>
  );
}
