import { defineConfig } from 'vite'
import swc from "unplugin-swc";

export default defineConfig({
    root: 'demo',
    plugins: [
        swc.vite({
            jsc: {
                experimental: {
                    plugins: [
                        [
                            "@effector/swc-plugin",
                            {
                                addLoc: true,
                                factories: ["./demo/src/factory"]
                            }
                        ]
                    ]
                }
            }
        })
    ]
})