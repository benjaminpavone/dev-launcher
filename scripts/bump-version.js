// Bumps patch version in package.json, tauri.conf.json, Cargo.toml and Cargo.lock in sync
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const root = dirname(dirname(fileURLToPath(import.meta.url)));

const pkgPath = join(root, "package.json");
const tauriPath = join(root, "src-tauri", "tauri.conf.json");
const cargoPath = join(root, "src-tauri", "Cargo.toml");
const cargoLockPath = join(root, "src-tauri", "Cargo.lock");

const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const tauri = JSON.parse(readFileSync(tauriPath, "utf8"));

const [major, minor, patch] = pkg.version.split(".").map(Number);
const newVersion = `${major}.${minor}.${patch + 1}`;

pkg.version = newVersion;
tauri.version = newVersion;

writeFileSync(pkgPath, JSON.stringify(pkg, null, "\t") + "\n");
writeFileSync(tauriPath, JSON.stringify(tauri, null, "\t") + "\n");

// Cargo.toml: the first `version = "..."` line is the one of the [package] section
const cargo = readFileSync(cargoPath, "utf8");
const cargoVersionRegex = /^version = "[^"]*"/m;
if (!cargoVersionRegex.test(cargo)) {
	console.error(`No version line found in ${cargoPath}`);
	process.exit(1);
}
writeFileSync(cargoPath, cargo.replace(cargoVersionRegex, `version = "${newVersion}"`));

// Cargo.lock: keep our own package entry in sync so the build does not leave it modified
const cargoName = cargo.match(/^name = "([^"]*)"/m)[1];
const cargoLock = readFileSync(cargoLockPath, "utf8");
const lockEntryRegex = new RegExp(`(name = "${cargoName}"\\r?\\nversion = ")[^"]*"`);
if (lockEntryRegex.test(cargoLock)) {
	writeFileSync(cargoLockPath, cargoLock.replace(lockEntryRegex, `$1${newVersion}"`));
}

console.log(`Version bumped to ${newVersion}`);
