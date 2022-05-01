import React from 'react';
import clsx from 'clsx';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import styles from './index.module.css';
// import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Ide from '@site/src/components/Ide';
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <Row>
          <Col>
            <img src="https://raw.githubusercontent.com/dada-lang/dada-artwork/main/dada.svg" width="600"></img>
          </Col>
          <Col>
            <blockquote class="rectangle-speech-border">
              <h1 class="dada-left-justify">Welcome to <b>Dada</b>, an experimental new programming language!</h1>

              <h3 class="dada-left-justify"><a href="/docs/dyn_tutorial">Care to try our live tutorial?</a></h3>
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
      description="Dada is a WIP programming language">

      <HomepageHeader />

      <main>
      </main>
    </Layout>
  );
}
