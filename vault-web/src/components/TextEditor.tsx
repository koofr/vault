import { css, cx } from '@emotion/css';
import * as monacoEditor from 'monaco-editor/esm/vs/editor/editor.api';
import { memo, useCallback } from 'react';

import { monacoLanguageForFileName } from '../utils/monacoLanguages';

import { LoadingCircle } from './LoadingCircle';
import { MonacoEditor } from './MonacoEditor';

export interface TextEditorProps {
  fileName: string;
  text: string;
  width: number;
  height: number;
}

export const TextEditor = memo<TextEditorProps>(
  ({ fileName, text, width, height }) => {
    const language = monacoLanguageForFileName(fileName);
    const editorDidMount = useCallback(
      (
        editor: monacoEditor.editor.IStandaloneCodeEditor,
        monaco: typeof monacoEditor
      ) => {
        // TODO save editor to a ref and focus on editing
        editor.focus();
      },
      []
    );
    const onChange = useCallback((newValue: string) => {}, []);
    const isEditing = false;
    const editorWidth = width - (isEditing ? 0 : 50);
    const editorHeight = height - (isEditing ? 0 : 65);

    return (
      <div
        className={cx(
          css`
            display: flex;
            flex-direction: column;
          `,
          !isEditing &&
            css`
              padding: 25px 25px 25px;
              justify-content: center;
              align-items: center;
              background-color: #f0f0f0;

              & .monaco-editor .view-lines {
                cursor: default;
              }

              & .monaco-editor .view-overlays .current-line {
                display: none;
              }

              & .monaco-editor .cursors-layer > .cursor {
                display: none !important;
              }

              & .monaco-editor .scroll-decoration {
                display: none;
              }
            `
        )}
        style={{
          width: `${width}px`,
          height: `${height}px`,
        }}
      >
        <div
          className={cx(
            css`
              display: flex;
              flex-direction: column;
              flex-grow: 1;
              width: 100%;
              overflow: hidden;
              background-color: #ffffff;
            `,
            !isEditing &&
              css`
                padding-top: 15px;
                box-shadow: 0 0px 4px 3px #d4d6d7;
              `
          )}
        >
          {text === undefined ? (
            <LoadingCircle />
          ) : (
            <MonacoEditor
              language={language}
              value={text}
              onChange={onChange}
              editorDidMount={editorDidMount}
              width={`${editorWidth}px`}
              height={`${editorHeight}px`}
              options={{
                fontSize: 13,
                readOnly: !isEditing,
                scrollbar: {
                  alwaysConsumeMouseWheel: true,
                },
                wordWrap: 'on',
                lineNumbers: isEditing ? undefined : 'off',
                minimap: {
                  enabled: isEditing,
                },
                wordBasedSuggestions: language !== 'plaintext',
                links: false,
              }}
            />
          )}
        </div>
      </div>
    );
  }
);
