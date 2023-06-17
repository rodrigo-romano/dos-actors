build: 
	docker build -t gmto.im/ngao-opm .
run:
	docker run --rm --env-file config.txt gmto.im/ngao-opm
push:
	aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin 378722409401.dkr.ecr.us-west-2.amazonaws.com
	docker tag gmto.im/ngao-opm:latest 378722409401.dkr.ecr.us-west-2.amazonaws.com/gmto.im/ngao-opm:latest
	docker push 378722409401.dkr.ecr.us-west-2.amazonaws.com/gmto.im/ngao-opm:latest
stack:
	aws s3 cp ngao-opm.yaml s3://gmto.modeling/stacks/
	aws cloudformation create-stack --stack-name ngao-opm --template-url https://s3.us-west-2.amazonaws.com/gmto.modeling/stacks/ngao-opm.yaml --region us-west-2