import fs from 'fs';
import path from 'path';
import { hash } from 'ohash';

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
const hash = sha.slice(0, 7)

const json = JSON.parse(path.join(__dirname, '../package.json'));

json.optionalDependencies = {
  "@kermanx/tree-shaker-win32-x64-msvc": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-win32-x64-msvc@${hash}`,
  "@kermanx/tree-shaker-darwin-x64": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-darwin-x64@${hash}`,
  "@kermanx/tree-shaker-linux-x64-gnu": `https://pkg.pr.new/KermanX/tree-shaker/@kermanx/tree-shaker-darwin-x64@${hash}`
};

fs.writeFileSync(path.join(__dirname, '../package.json'), JSON.stringify(json, null, 2));
