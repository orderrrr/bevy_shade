<script>
  import CodeMirror from "svelte-codemirror-editor";
  import { javascript } from "@codemirror/lang-javascript";

  let value = "";
</script>

<CodeMirror bind:value lang={javascript()} />
