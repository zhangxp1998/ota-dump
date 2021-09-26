# Rust learning project

update_metadata.proto is copied from [aosp](https://cs.android.com/android/platform/superproject/+/master:system/update_engine/update_metadata.proto;l=1?q=update_metadata.proto&sq=)

# Usage

`ota-dump <path to android OTA.zip>`


# Examples

* `ota-dump coral_ota.zip`

  Dump the entire OTA manifest in json format. Dumped object has type 
[DeltaArchiveManifest](https://cs.android.com/android/platform/superproject/+/master:system/update_engine/update_metadata.proto;l=396?q=DeltaArchive&sq=).
This object is huge so it's best to save it to a file or pipe to other CLI tools such as [jq](https://stedolan.github.io/jq/)
* `ota-dump cf_x86_dm_verity.zip | jq '.partitions[].partition_name'`

   will list partitions included in this update.

* `ota-dump cf_x86_dm_verity.zip | jq '{name: .partitions[].partition_name, size: .partitions[].new_partition_info.size}'` 
  
	List partitions included and size of partitions after OTA update.
* `ota-dump cf_x86_dm_verity.zip | jq 'del(.partitions[].operations)|del(.partitions[].merge_operations)'`

  Dump the manifest without operation list without list of operations. The output will be much smaller.