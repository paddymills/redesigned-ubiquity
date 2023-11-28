
# SAP Consumption

This executable pulls data regarding programs that were burned in the previous timeframe and pushes them to SAP

## Install/Uninstall

The executable handles all functions and works as its own installer/uninstaller.

The reason for the install/uninstall pertains to the usage of the Windows Event logs as a logging mechanism. `install` will register the application with the event logger and `uninstall` will un-register it. These commands make calls to the Windows registry, so these commands must be run with elevated privileges as mentioned.

### Install process

1) Place the binary (`sap_consumption.exe`) at the file location that it will reside.
2) Run 'sap_consumption.exe generate-config'. This will generate the `config.toml` file in the location from step 1.
3) Edit `config.toml` from step 2.
    - output_dir: The network path to where the files will be written to.
    - logging_name: The application name used in the Windows Event Logger.
    - database: The server and database of the Sigmanest database.
4) Open a terminal or command prompt as Administrator and run 'sap_consumption.exe install' to register the application with the logger.
5) Make a Windows scheduled task to run the executable on a planned interval (e.g. every hour)

### Permissions

The user that the scheduled task is ran as needs to have the following permissions
- Read/Write access to the database in the config
- Write access to the output directory in the config
- Read/Write access to the folder where the executable is placed

### Uninstall process

Open a terminal or command prompt as Administrator and run 'sap_consumption.exe uninstall' to unregister the application from the logger.
