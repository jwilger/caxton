#!/bin/bash
cd /home/jwilger/projects/caxton
npx markdownlint docs/**/*.md README.md ARCHITECTURE.md 2>&1 | head -200
