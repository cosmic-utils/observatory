<div align="center">
  <img src="res/icons/hicolor/scalable/apps/icon.svg" width="300" />
  <h1>Observatory</h1>
  <p>An in-development system monitor application for the COSMIC™ desktop environment</p>
  
  <br/><br/>

  ![Screenshot of the observatory app's processes page](observatory/res/screenshots/disk-light.png#gh-light-mode-only)
  ![Screenshot of the observatory app's processes page](observatory/res/screenshots/disk-dark.png#gh-dark-mode-only)

</div>

# Features

View information about your system.

## CPU

View usage per core, threads, processes handles, up time, speed and total usage.

![CPU Light](observatory/res/screenshots/processor-light.png#gh-light-mode-only)
![CPU Dark](observatory/res/screenshots/processor-dark.png#gh-dark-mode-only)

## Memory

View total memory and swap usage.

![Memory Light](observatory/res/screenshots/memory-light.png#gh-light-mode-only)
![Memory Dark](observatory/res/screenshots/memory-dark.png#gh-dark-mode-only)

## Disk usage

View usage per disk and total read and write.

![Disk Light](observatory/res/screenshots/disk-light.png#gh-light-mode-only)
![Disk Dark](observatory/res/screenshots/disk-dark.png#gh-dark-mode-only)

## Processes

View and manage running processes on your system.

![Processes Light](observatory/res/screenshots/processes-light.png#gh-light-mode-only)
![Processes Dark](observatory/res/screenshots/processes-dark.png#gh-dark-mode-only)

## Installation

To install, clone this repository and run the following commands:

```
just build-release
sudo just install
```

make sure you have `just` installed on your system
