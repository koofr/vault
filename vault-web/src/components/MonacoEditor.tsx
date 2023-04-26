import 'monaco-editor/esm/vs/editor/editor.main';
import MonacoEditor from 'react-monaco-editor';

// eslint-disable-next-line no-restricted-globals
self.MonacoEnvironment = {
  getWorker(_, label) {
    if (label === 'css' || label === 'scss' || label === 'sass') {
      return import('monaco-editor/esm/vs/language/css/css.worker?worker').then(
        (mod) => new mod.default()
      );
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return import(
        'monaco-editor/esm/vs/language/html/html.worker?worker'
      ).then((mod) => new mod.default());
    }
    if (label === 'json') {
      return import(
        'monaco-editor/esm/vs/language/json/json.worker?worker'
      ).then((mod) => new mod.default());
    }
    if (label === 'typescript' || label === 'javascript') {
      return import(
        'monaco-editor/esm/vs/language/typescript/ts.worker?worker'
      ).then((mod) => new mod.default());
    }
    return import('monaco-editor/esm/vs/editor/editor.worker?worker').then(
      (mod) => new mod.default()
    );
  },
};

export { MonacoEditor };
