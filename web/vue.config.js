const path = require('path');
const webpack = require('webpack');

module.exports = {
  configureWebpack: {
    output: {
      clean: false,
      filename: '[name].js',
      chunkFilename: 'js/[name].js',
    },
    plugins: [
      new webpack.DefinePlugin({
        'process.env.VUE_APP_BUILD_TYPE': JSON.stringify(process.env.VUE_APP_BUILD_TYPE),
      }),
    ],
    devServer: {
      historyApiFallback: true,
      proxy: {
        '^/api': {
          target: 'http://localhost:3000',
        },
      },
    },
  },
  css: {
    extract: {
      filename: '[name].css',
      chunkFilename: 'css/[name].css',
    },
  },
  chainWebpack: (config) => {
    config.plugin('html')
      .tap((args) => {
        // eslint-disable-next-line no-param-reassign
        args[0].minify = false;
        return args;
      });
  },
  transpileDependencies: [
    'vuetify',
  ],
  publicPath: './',
  // path.resolve избегает бага html-webpack-plugin на Windows (pub lic → public)
  outputDir: path.resolve(__dirname, 'public'),
  indexPath: 'index.html',
  filenameHashing: false,
};
