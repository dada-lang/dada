/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import CodeBlock from '@theme-init/CodeBlock';
import BrowserOnly from '@docusaurus/BrowserOnly';

const withDadaEditor = () => {
  function WrappedComponent(props) {
    if (props.ide) {
      return <BrowserOnly fallback={<div>Loading...</div>}>
        {() => {
          const Ide = require('@site/src/components/Ide').default;
          return <Ide mini={true} sourceText={props.children} />;
        }}
      </BrowserOnly>;
    }
    return <CodeBlock {...props} />;
  }

  return WrappedComponent;
};

export default withDadaEditor();
