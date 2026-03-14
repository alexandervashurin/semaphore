const { src, dest, series, parallel, watch } = require('gulp');
const rename = require('gulp-rename');
const sass = require('gulp-sass')(require('sass'));
const terser = require('gulp-terser');
const concat = require('gulp-concat');
const cleanCss = require('gulp-clean-css');
require('dotenv').config();
const gptTranslate = require('./gulp-gpt-translate');

const LANG_NAMES = {
  en: 'English',
  ru: 'Russian',
  es: 'Spanish',
  fr: 'French',
  de: 'German',
  it: 'Italian',
  ja: 'Japanese',
  ko: 'Korean',
  pt: 'Portuguese',
  zh_cn: 'Simplified Chinese',
  zh_tw: 'Traditional Chinese',
  nl: 'Dutch (Netherlands)',
  pl: 'Polish',
  pt_br: 'Brazilian Portuguese',
};

// ==================== Vue CLI Tasks ====================

function tr() {
  return Object.keys(LANG_NAMES).filter((lang) => lang !== 'en').map((lang) => src('src/lang/en.js')
    .pipe(gptTranslate({
      apiKey: process.env.OPENAI_API_KEY,
      targetLanguage: LANG_NAMES[lang],
      messages: [
        'Translate values of the JS object fields.',
        'Preserve file format. Do not wrap result to markdown tag. Result must be valid js file.',
      ],
    }))
    .pipe(rename({ basename: lang }))
    .pipe(dest('src/lang')));
}

// ==================== Vanilla JS Tasks ====================

// Пути для ванильного фронтенда
const paths = {
  vanilla: {
    scss: 'vanilla/css/**/*.scss',
    js: 'vanilla/js/**/*.js',
    html: 'vanilla/html/**/*.html',
    assets: 'vanilla/assets/**/*',
    src: 'vanilla/',
    dest: 'public/'
  }
};

// SCSS → CSS для ванильного фронтенда
function vanillaStyles() {
  return src('vanilla/css/main.scss')
    .pipe(sass().on('error', sass.logError))
    .pipe(cleanCss())
    .pipe(rename('main.min.css'))
    .pipe(dest(paths.vanilla.dest + 'css'));
}

// JS минификация для ванильного фронтенда
function vanillaScripts() {
  return src('vanilla/js/app.js')
    .pipe(terser())
    .pipe(rename('app.min.js'))
    .pipe(dest(paths.vanilla.dest + 'js'));
}

// Копирование HTML для ванильного фронтенда
function vanillaHtml() {
  return src('vanilla/html/**/*.html')
    .pipe(dest(paths.vanilla.dest + 'html'));
}

// Копирование ассетов
function vanillaAssets() {
  return src('vanilla/assets/**/*', { encoding: false })
    .pipe(dest(paths.vanilla.dest + 'assets'));
}

// Watch для ванильного фронтенда
function vanillaWatch() {
  watch(paths.vanilla.scss, vanillaStyles);
  watch(paths.vanilla.js, vanillaScripts);
  watch(paths.vanilla.html, vanillaHtml);
}

// Build для ванильного фронтенда
const vanillaBuild = series(
  parallel(vanillaStyles, vanillaScripts, vanillaHtml, vanillaAssets)
);

// Default task для ванильного фронтенда
const vanillaDefault = series(vanillaBuild, vanillaWatch);

// ==================== Экспорт задач ====================

module.exports = {
  tr,
  // Vanilla JS задачи
  vanillaStyles,
  vanillaScripts,
  vanillaHtml,
  vanillaAssets,
  vanillaWatch,
  vanillaBuild,
  default: vanillaDefault,
};
