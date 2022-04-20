# Dada Web React

This is a webapp for playing with Dada, built using [React](https://reactjs.org)/[Redux](https://redux-toolkit.js.org).

# Installation / Running

You need to have a [node](https://nodejs.org/) version 16 environment available. With that, you can type

```
npm install .
npm start
```

And then you are ready to go!

# Craco, react-scripts, mdx, oh my!

In order to get mdx working with react-scripts-5.0.0, some annoying things were necessary:

* There is [a bug](https://github.com/mdx-js/mdx/issues/2004) in react-scripts-5.0.1 that makes it not work with mdx.
* As documented [here](https://github.com/mdx-js/mdx/pull/2010/), the current workaround is to use a [craco] configuration to override part of react-scripts.
* Annoyingly, craco is not officially compatible with react-scripts 5.x, so when you do `npm update` you get warnings: the fix is to use `npm update --force`.

[craco]: https://github.com/gsoft-inc/craco

# TODO

- Replace the logo and favicon with the Dada logo
