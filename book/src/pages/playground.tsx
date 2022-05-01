import React from 'react';
import Layout from '@theme/Layout';
import Ide from '@site/src/components/Ide';

export default function Playground(): JSX.Element {
    return (
        <Layout
            title="Dada playground"
            description="Dada playground">

            <main>
                <Ide source='async fn main() {\n    print("Hello, world!").await\n}\n'></Ide>
            </main>
        </Layout>
    );
}
