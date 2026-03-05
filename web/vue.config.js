const webpack = require('webpack');

module.exports = {
  configureWebpack: {
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
    output: {
      filename: 'app.js',
      chunkFilename: 'js/[name].js'
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
  // Публичный путь - используем относительные пути для локальной работы
  publicPath: './',
  // Собираем в web/public для локальной раздачи через Rust-сервер
  outputDir: '../web/public',
  // Имя файла HTML
  indexPath: 'index.html',
  // Отключаем хеширование для предсказуемых имен файлов
  filenameHashing: false,
  css: {
    extract: {
      filename: 'app.css'
    }
  },
};
