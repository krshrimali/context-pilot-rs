## Current design and TODOs

### Design ideas

1. Make sure that everything can run in the background - asynchronous
2. Keep the server running, and iterate through all the active buffers first; and then all the files in the current file to find the most relevant files there.
   - Check if the current file has been iterated before, if yes:
     - Return the result
   - If not:
     - prioritse this over the init function and do this instead?
     - and store the result
   - For this, maintain a priority queue maybe?
3. Store everything in in-house DB and make sure that it's persisted in the DB on exit / reboot.

A user should have option for some configs:

1. Number of relevant files and authors, this goes in extension config. On each config change, the user should call initialize function to be able to change it.
2. Can each workspace have an optional config where they know which files / extensions / directories to index? OR which directories or files with extensions to NOT index?

### Open questions

1. Can we yield instead of return?
2. can we let user know on the progress?
3. In case file is not present, the file should have "renamed" flag.
4. Once they click on a file, and if the flag `renamed` is somehow associated, call the server back again to fetch the new file.
5. What's the best way to _almost sort_ given array?
6. How to calculate available threads?

### Design on Notes

1. It's more like a scheduler design initially where load balancing has to happen across available threads
