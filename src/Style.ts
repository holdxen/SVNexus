import { css } from "@linaria/core";

export const globalStyles = css`
  :global() {
    body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      color: var(--semi-color-text-0);
      background-color: var(--semi-color-bg-0);
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

    * {
      scrollbar-width: thin;
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
      --os-size: 10px;
      --scrollbar-size: 10px;
    }

  }
`;
