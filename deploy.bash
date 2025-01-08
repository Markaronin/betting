#!/bin/bash
set -eu

if [ $# -eq 0 ]
  then
    echo "No arguments supplied"
fi

s3_bucket=$1

cargo lambda build --release

lambda_name=betting
output_path="./target/lambda/$lambda_name/bootstrap"

temp_dir=$(mktemp -d)
zip_file_path="$temp_dir/function.zip"
lambda_path="$temp_dir/bootstrap"

zip --junk-paths $zip_file_path $output_path > /dev/null

s3_file_name="$(uuid).zip"
s3_path="s3://${s3_bucket}/${s3_file_name}"

aws s3 cp "${zip_file_path}" "${s3_path}"
aws lambda update-function-code --function-name "${lambda_name}" --s3-bucket "${s3_bucket}" --s3-key "${s3_file_name}" > /dev/null
aws s3 rm "${s3_path}"

rm -r $temp_dir

echo "Finished uploading $lambda_name"