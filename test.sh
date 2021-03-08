for dir in ./commander ./peregrine-core ./identitynumber ./peregrine-dauphin ./dauphin-interp-toy ./peregrine-dauphin-queue ./keyed ./dauphin ./dauphin-test-harness ./varea ./dauphin-lib-buildtime ./peregrine-web ./dauphin-lib-std ./dauphin-compile ./dauphin-interp ./blackbox ./dauphin-lib-peregrine; do
    (
        echo "Running tests in $dir"
        cd $dir
        cargo test || exit
    ) || exit
done
