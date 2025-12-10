# PDM

## TODO
- [x] Start a new repo: https://github.com/p2poolv2/pdm
- [x] Start a new project using ratatui
- [ ] Add a button to select bitcoind config file.
- [ ] On click of the button show a tui file explorer that allows user to select bitcoin.conf file
- [ ] Parse the conf file - make sure to use proper types for each field. Also 
add helpful messages like https://jlopp.github.io/bitcoin-core-config-generator/
- [ ] Divide the view into sections - as lopp's generator does.
- [ ] Let user edit any field. No need to add validation here.
- [ ] Provide a Save button, so user can save the file.
- [ ] Check if the pidfile config provided by the user points to a pid file that exists. If it exists, check if a process with the pid exists. Show an indicator, let's call it health indicator that shows if the process is running.