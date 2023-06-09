crate:
	cd .. ;  git archive --format=tar.gz HEAD > docker/crate.tar.gz
build: 
	docker build -t gmto.im/asms/trfun-an .
run:
	docker run --rm --env-file config.txt gmto.im/gmto.im/asms/trfun-an
push:
	aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin 378722409401.dkr.ecr.us-east-2.amazonaws.com
	docker tag gmto.im/asms/trfun-an:latest 378722409401.dkr.ecr.us-east-2.amazonaws.com/gmto.im/asms/trfun-an:latest
	docker push 378722409401.dkr.ecr.us-east-2.amazonaws.com/gmto.im/asms/trfun-an:latest
stack:
	aws s3 cp asms.yaml s3://gmto.modeling/stacks/
	aws cloudformation create-stack --stack-name asms --template-url https://s3.us-west-2.amazonaws.com/gmto.modeling/stacks/asms.yaml --region us-east-2