#!/bin/bash
echo '------------------------------------------------------------------------------------------'
echo 'Copyright @2020 for Wushi (Fujian) Technology Co., Ltd'
echo 'This shell script will query installed mix agent'
echo "Current working directory is: $(pwd)"
echo '------------------------------------------------------------------------------------------'

c=$(ps aux | grep -v grep | grep -c mix_agent)
if [ "$c" -eq 0 ]
 then
  echo "No any agent install"
else
  ps aux | grep -v grep | grep mix_agent
fi