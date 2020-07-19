
# Recut
A utf-8, regular expression delimiter implementation of the UNIX tool CUT. With built in smart features inspired by Ripgrep and Search and Displace (SD).


## Features include

 - UTF-8 Support.
 - Regex Delimiters.
 - Inferring Delimiter based on first line.
 - Ranges e.g "5:" means print everything from the fifth position on wards based on the first line.
 - Index from the end using the "-" symbol e.g. "-2" means second to last item base don the first line.
 - Quoted data counts as a single line similar to a CSV  (with double quotes as an escape).
 

## Features coming soon

 - infer position to print based on first line.


## Current state
Please note this is not ready for actual use yet. Currently debug messages are still being printed. Code is not really commented as there is lots of functions being duplicated with no decision being made on whether the functions are similar enough to be abstracted away. 

Inferring the delimiter only does single characters, it will never infer delimiters such as `\w+`. This may change in the future to handle multiple tabs and spaces as one regex delimiter.

Each function prints the results, this is likely to change to sending data out using a channel. This will still allow handling of the data as a stream and will allow multi-threading so output does not slow down the data processing. 

Bytes and characters have been implemented by not tested much


## Examples of features




