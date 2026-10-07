import { css } from '@linaria/core'

export const globalStyles = css`
  :global() {
    body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      color: var(--semi-color-text-0);
      background-color: var(--semi-color-bg-0);
    }

    /* Semi 在暗色下用 body[theme-mode="dark"] 声明了 Inter 字体栈，特异性高于这里的 body，
       所以选择器必须跟着提一级，靠后加载才能压住它 */
    body,
    body[theme-mode="dark"] {
      font-family:
        "JetBrains Mono",
        -apple-system,
        BlinkMacSystemFont,
        "Segoe UI",
        Roboto,
        sans-serif;
    }

    html {
      width: 100vw;
      height: 100vh;
    }

    #root {
      width: 100%;
      height: 100%;
    }

    body {
      ---svnexus-list-item-default-background: transparent;
      ---svnexus-list-item-hover-background: var(--semi-color-fill-1);
      ---svnexus-list-item-pressed-background: var(--semi-color-fill-2);
      ---svnexus-list-item-selected-background: rgba(var(--semi-blue-2), 1);
      ---svnexus-list-item-selected-hover-background: rgba(
        var(--semi-blue-3),
        1
      );
      ---svnexus-list-item-disabled-background: transparent;
      ---svnexus-list-item-disabled-selected-background: var(
        --semi-color-fill-0
      );
    }

    body {
      user-select: none;
      overflow: hidden;
      --svnexus-base-color: #ebebeb;
      --scrollbar-color: rgba(0, 0, 0, 0.2);
    }
    body[theme-mode="dark"] {
      --svnexus-base-color: rgba(var(--semi-grey-1), 1);
      --scrollbar-color: rgba(255, 255, 255, 0.2);
      /* Semi 明暗用的是同一个遮罩值 rgba(22, 22, 26, .6)，而它正好等于暗色的内容底色，
         压在面板上等于没压；换成纯黑加重，弹窗才和背景分得开 */
      --semi-color-overlay-bg: rgba(0, 0, 0, 0.6);
    }
    * {
      scrollbar-color: var(--scrollbar-color) transparent;
      scrollbar-width: thin;
    }
  }
`
