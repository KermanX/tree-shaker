import fs from 'fs';
import path from 'path';
import { hash } from 'ohash';
import { fileURLToPath } from 'url';

const apiUrl = "https://pkg.pr.new/"

const {
  GITHUB_REPOSITORY,
  GITHUB_RUN_ID,
  GITHUB_RUN_ATTEMPT,
  GITHUB_ACTOR_ID,
} = process.env;

const [owner, repo] = GITHUB_REPOSITORY.split("/");

const metadata = {
  owner,
  repo,
  run: Number(GITHUB_RUN_ID),
  attempt: Number(GITHUB_RUN_ATTEMPT),
  actor: Number(GITHUB_ACTOR_ID),
};

const key = hash(metadata);

const checkResponse = await fetch(new URL("/check", apiUrl), {
  method: "POST",
  body: JSON.stringify({
    owner,
    repo,
    key,
  }),
});

if (!checkResponse.ok) {
  console.error(await checkResponse.text());
  process.exit(1);
}

const { sha } = await checkResponse.json();
const tag = sha.slice(0, 7)

const jsonPath = path.join(fileURLToPath(import.meta.url), '../../package.json');
const json = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));

json.optionalDependencies = {
  "@kermanx/tree-shaker-win32-x64-msvc": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-win32-x64-msvc@${tag}`,
  "@kermanx/tree-shaker-darwin-x64": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-darwin-x64@${tag}`,
  "@kermanx/tree-shaker-darwin-arm64": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-darwin-arm64@${tag}`,
  "@kermanx/tree-shaker-linux-x64-gnu": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-darwin-x64@${tag}`
};

fs.writeFileSync(jsonPath, JSON.stringify(json, null, 2));
