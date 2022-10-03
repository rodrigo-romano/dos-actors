build:
	docker build -t gmto.im/domeseeing .
run:
	docker run --gpus all --rm gmto.im/domeseeing
push:
	aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin 378722409401.dkr.ecr.us-west-2.amazonaws.com
	docker tag gmto.im/domeseeing:latest 378722409401.dkr.ecr.us-west-2.amazonaws.com/gmto.im/domeseeing:latest
	docker push 378722409401.dkr.ecr.us-west-2.amazonaws.com/gmto.im/domeseeing:latest
stack:
	aws s3 cp domeseeing.yaml s3://gmto.modeling/stacks/
	aws cloudformation create-stack --stack-name domeseeing --template-url https://s3-us-west-2.amazonaws.com/gmto.modeling/stacks/domeseeing.yaml --region us-west-2
