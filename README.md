## Rail hourly departures
Get the number of departures for each national rail station in Great Britain at each hour.

## Steps to run with new data
1. Install Rust on your machine 
```
curl -sSf https://sh.rustup.rs | sh
```
1. Download the Network Rail timetable cif file (via [Rail Data Marketplace](https://raildata.org.uk/dataProduct/P-dbd92416-2f09-4f72-ad42-d53bbfec50f3/overview))

1. Create a list of the GB station three-alpha codes

1. Update the file path to the directory with rail timetables and three-alpha codes (line 8 in run.sh)

1. Select a day and week (lines 9-10 in run.sh) to calculate for

1. Run the script
```
bash run.sh
```