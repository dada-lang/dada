/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import React from 'react';
import CodeBlock from '@theme-init/CodeBlock';
import Ide from '@site/src/components/Ide';

const withDadaEditor = () => {
  function WrappedComponent(props) {
    console.log(`WrappedContent: ${JSON.stringify(props)}`);
    if (props.ide) {
      return <Ide sourceText={props.children} />;
    }
    return <CodeBlock {...props} />;
  }

  return WrappedComponent;
};

export default withDadaEditor();
