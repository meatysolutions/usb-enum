const { copyFileSync, existsSync } = require('fs');
const { platform, arch } = require('os');
const { platformArchTriples } = require('@napi-rs/triples');
const { join } = require('path');

const triples = platformArchTriples[platform()][arch()];
const tripe = triples[0];
const SRC_FILE = `usb-enum.${tripe.platformArchABI}.node`;
const DST_FILE = './usb-enum.node';

const publishSrc = join('./artifacts', SRC_FILE);
if (existsSync(publishSrc)) {
  copyFileSync(publishSrc, DST_FILE);
  return;
}

const devSrc = join('./', SRC_FILE);
if (existsSync(devSrc)) {
  copyFileSync(devSrc, DST_FILE);
  return;
}
