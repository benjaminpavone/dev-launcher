// Renames the built exe to dev-launcher_<version>.exe after tauri build
import { readFileSync, renameSync, existsSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = pkg.version;

const releaseDir = join(root, "src-tauri", "target", "release");
const src = join(releaseDir, "dev-launcher.exe");
const dest = join(releaseDir, `dev-launcher_${version}.exe`);

if (!existsSync(src)) {
	console.error(`Exe not found: ${src}`);
	process.exit(1);
}

renameSync(src, dest);
console.log(`Renamed to dev-launcher_${version}.exe`);
