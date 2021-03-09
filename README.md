# PhotoSort


## Command line logic

```
$ photosort (move|copy) SOURCE_DIR DEST_DIR [--verbose] [--dry-run] [--log-file LOG_FILE]
```

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