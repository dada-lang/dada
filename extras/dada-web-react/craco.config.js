const { addAfterLoader, loaderByName } = require('@craco/craco')
// module.exports = {
//     webpack: {
//         configure: (webpackConfig) => {
//             addAfterLoader(webpackConfig, loaderByName('babel-loader'), {
//                 test: /\.(md|mdx)$/,
//                 loader: require.resolve('@mdx-js/loader')
//             })
//             return webpackConfig
//         }
//     }
// }