# Building Your First Complete SEA Model

In this tutorial, we will build a comprehensive domain model for a fictional E-Commerce system. We will cover Entities, Resources, Flows, and Instances.

## The Scenario

We are modeling "ShopEasy", an online store. It has:
- A **Storefront** (Web App)
- An **Order Service** (Backend API)
- A **Payment Gateway** (External Service)
- An **Order Database** (Storage)

## Step 1: Define Entities

Create a file `shopeasy.sea`. Start by defining the active components.

```sea
entity Storefront {
    type = "service"
    language = "typescript"
    public = true
}

entity OrderService {
    type = "service"
    language = "rust"
    layer = "backend"
}

entity PaymentGateway {
    type = "external_service"
    provider = "stripe"
}
```

## Step 2: Define Resources

Now define the passive infrastructure.

```sea
resource OrderDB {
    type = "database"
    engine = "postgres"
    encrypted = true
}

resource EmailQueue {
    type = "queue"
    engine = "rabbitmq"
}
```

## Step 3: Connect with Flows

Model how these components interact.

```sea
flow place_order {
    from = Storefront
    to = OrderService
    interaction = "http_post"
    payload = "OrderRequest"
}

flow save_order {
    from = OrderService
    to = OrderDB
    interaction = "sql_insert"
}

flow process_payment {
    from = OrderService
    to = PaymentGateway
    interaction = "api_call"
}

flow queue_confirmation {
    from = OrderService
    to = EmailQueue
    interaction = "publish"
}
```

## Step 4: Define Instances

Model the physical deployment in the production environment.

```sea
instance prod_order_service {
    of = OrderService
    env = "production"
    replicas = 3
}

instance prod_db {
    of = OrderDB
    env = "production"
    region = "us-west-2"
}
```

## Step 5: Validate

Run the CLI to verify your model structure.

```bash
sea-cli parse shopeasy.sea
```

## Understanding the Output

The parser confirms that:
1. All references are valid (e.g., `from = Storefront` refers to a defined Entity).
2. The syntax is correct.
3. The graph is connected.

## Conclusion

You have successfully modeled a microservices architecture with data flows and infrastructure definitions. This model can now be used to generate diagrams, Terraform code, or validate security policies.

## See Also

- [Semantic Modeling Concepts](../explanations/semantic-modeling-concepts.md)
