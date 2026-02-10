#!/bin/bash



export DYLD_LIBRARY_PATH=$(realpath ./svn/lib):$DYLD_LIBRARY_PATH
export PATH=$(realpath ./svn/bin):$PATH

svn --version
