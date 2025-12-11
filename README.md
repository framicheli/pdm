# PDM

## TODO

### Version 1

- [x] Start a new repo: https://github.com/p2poolv2/pdm
- [x] Start a new project using ratatui
- [x] Add a button to select bitcoind config file.
- [x] On click of the button show a tui file explorer that allows user to select bitcoin.conf file
- [x] Parse the conf file - make sure to use proper types for each field. Also 
add helpful messages like https://jlopp.github.io/bitcoin-core-config-generator/
- [x] Divide the view into sections - as lopp's generator does.
- [x] Let user edit any field. No need to add validation here.
- [x] Provide a Save button, so user can save the file.
- [x] Check if the pidfile config provided by the user points to a pid file that exists. If it exists, check if a process with the pid exists. Show an indicator, let's call it health indicator that shows if the process is running.


### Version 2

- [ ] Check how much disk space is consumed by bitcoin datadir
- [ ] Check how much total disk is available and how much is used up. Show the pipes progress bar for this - use existing ratatui widget for this.
- [ ] Show chain info - current height, etc.
- [ ] Show number of connections - on clicking on this, we should show all IPs we are connected to - use https://developer.bitcoin.org/reference/rpc/getpeerinfo.html
- [ ] Show memory used by bitcoin process - like htop does for system.
- [ ] Show tail of the bitcoin log