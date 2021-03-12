# PhotoSort

PhotoSort is a simple project allowing you to sort your photos into dated directories in form `YYYY/MM/DD`. 

## How to build photosort

Photosort works well on Linux and may work on Mac. Below instructions are for Linux:

```
# Clone photosort

$ git clone https://github.com/sgzmd/photosort.git
$ cd photosort
```

In order to sort videos, you need to have `ffmpeg` and corresponding development libraries installed. On Debian-based systems you can install them as follows:

```
$ sudo apt install libavutil-dev libavformat-dev libavfilter-dev \
    libavformat-dev libclang-dev libavdevice-dev ffmpeg
```

After that photosort should build using standard `cargo` commands:

```
$ cargo build --release
```

## Using photosort

Photosort has two modes of work: working with regular directories of files and zip files. To run photosort on directory of files:

```
$ ./photosort --src=<PATH_TO_YOUR_FILES> --dest=<WHERE_ROOT_DIRECTORY_SHOULD_BE> \
   --mode=copy --log=output.log
```

To move files instead of copying them (much faster) use `--mode=move`. If you omit log, nothing will be logged.

To run photosort on zip file, simply specify `--src=/path/to/zip/file.zip`.   


## Attributions

PhotoSort is relying on [exif-samples](https://github.com/ianare/exif-samples) github 
repo for testing exif reading / fs walking functionality.

## Requires

  * libavutil
  * libavformat
  * libavfilter
  * libavformat
  * libclang
  * libavdevice
