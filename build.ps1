param ($tagname='latest')

$imagename="ivolchenkov/wednesday:$tagname"

write-host "Building image with tag '$imagename'"

cross build --target x86_64-unknown-linux-gnu --release
docker build -t $imagename .
docker push $imagename