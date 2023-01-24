# fedup

file deduplicator, written in rust as a learning exercise

## How it should work

Fedup should recursively scan a list of folders and collect info about files paired with some hints about their contents. While this file collection is being built, the app will essentially try matching identical files which have been already scanned and group them together.

For these initial equality checks fedup will build an initial fingerprint of a file as being just the size of the file. So files with identical sizes will form a group.

Next, these groups of equally sized files will be investigated further for content equality. To avoid parsing entire files, we'll build a group-specific fingerprint which will now consist of a SHA-256 hash of fixed sample of bytes sample from all the group members from a random offset. Since all files I'll be using here for deduplication will be JPEG images, this trick should be sufficient. Again, files with the same content fingerprint will be (sub-)grouped together.

At this stage, groups with more than one entry will represent the duplicates. A list of safe-to-delete files will be provided by selecting from each duplicates group all its members except the first one.

This list should be provided in either json format or a simple flat file of file paths on each line.

## Implementation requirements

The aim is to produce a multithreaded application.

Since is this is a toy app, stopping/resuming (which would imply saving state) is not really a necessary feature. And the input file folders are assumed to be static so we don't have to watch them being update while an ongoing scan takes place.
