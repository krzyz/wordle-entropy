import './style.scss';

import init, {run_app} from "./pkg"

async function run() {
  await init();
  run_app();
}

run()

/*
import("./pkg").then(module => {
  module.run_app();
});
*/
