#!/usr/bin/env node

import cheerio from 'cheerio';
import fs from 'fs';
import { optimize } from 'svgo';

const svgPath = process.argv[2];

if (!svgPath) {
  console.error('Please provide the path to the SVG file.');
  process.exit(1);
}

let svgString = fs.readFileSync(svgPath);

// Run svgo
svgString = optimize(svgString, {
  multipass: true,
}).data;

const svg = cheerio.load(svgString);

// Remove all clipPath elements (svg('clipPath').remove() does not work)
Array.from(svg('*'))
  .filter((el) => el.name === 'clipPath')
  .forEach((el) => {
    svg(el).remove();
  });

// Remove all clip-path attributes from other elements
svg('*').removeAttr('clip-path');

svgString = svg('body').html();

// Run svgo again
svgString = optimize(svgString, {
  multipass: true,
}).data;

fs.writeFileSync(svgPath, svgString);

console.log(
  `${svgPath}: optimized svg, removed clipPath elements and clip-path attributes.`
);
