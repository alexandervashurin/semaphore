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
  transpileDependencies: [
    'vuetify',
  ],
  publicPath: './',
  // Используем стандартную сборку без pages
  outputDir: path.resolve(__dirname, 'dist'),
  indexPath: 'index.html',
  filenameHashing: false,
};
