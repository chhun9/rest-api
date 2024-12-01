import { defineConfig } from "vite";
import AutoImport from 'unplugin-auto-import/vite';
import vue from "@vitejs/plugin-vue";
import path from 'path'

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue(),
  AutoImport({
    imports: ['vue'], // vue의 ref, reactive, computed 등을 자동으로 가져옴
    dts: 'src/auto-imports.d.ts', // 자동 생성된 타입 선언 파일
  }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'), // '@'를 src 디렉토리로 매핑
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
