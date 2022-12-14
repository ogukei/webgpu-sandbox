const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
    mode: 'production',
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.[contenthash].js',
    },
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'webassembly/sync',
            }
        ],
        rules: [
            {
                test: /\.ts$/,
                use: 'ts-loader',
                exclude: [
                    /node_modules/,
                    /pkg/
                ]
            },
        ],
    },
    experiments: {
        syncWebAssembly: true
    },
    performance: {
        hints: false,
        maxAssetSize: 5 * 1024 * 1024
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: 'index.html'
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".")
        }),
        new CopyWebpackPlugin({
            patterns: [
                { from: 'coi-serviceworker/coi-serviceworker.min.js', context: 'node_modules' },
                { from: 'bulma/css/bulma.min.css', context: 'node_modules' },
            ]
        })
    ]
};

// https://webpack.js.org/guides/typescript/
// https://webpack.js.org/plugins/copy-webpack-plugin/
// https://github.com/gzuidhof/coi-serviceworker
