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
@namespace "shopeasy"

Entity "Storefront"
Entity "OrderService"
Entity "PaymentGateway"
Entity "OrderDatabase"
```

## Step 2: Define Resources

Now define the passive infrastructure.

```sea
Resource "OrderRequest" units
Resource "OrderRecord" units
Resource "PaymentIntent" units
Resource "ConfirmationMessage" units
```

## Step 3: Connect with Flows

Model how these components interact.

```sea
Flow "OrderRequest" from "Storefront" to "OrderService"
Flow "OrderRecord" from "OrderService" to "OrderDatabase"
Flow "PaymentIntent" from "OrderService" to "PaymentGateway"
Flow "ConfirmationMessage" from "OrderService" to "Storefront"
```

## Step 4: Define Instances

Model the physical deployment in the production environment.

```sea
Instance prod_order_service of "OrderService" {
    env: "production",
    replicas: 3
}

Instance prod_db of "OrderDatabase" {
    env: "production",
    region: "us-west-2"
}
```

## Step 5: Validate

Run the CLI to verify your model structure.

```bash
sea-cli parse shopeasy.sea
```

## Understanding the Output

The parser confirms that:
1. All references are valid (e.g., `Flow "OrderRequest" from "Storefront"` points to declared entities).
2. The syntax is correct.
3. The graph is connected.

## Conclusion

You have successfully modeled a microservices architecture with data flows and infrastructure definitions. This model can now be used to generate diagrams, Terraform code, or validate security policies.

## See Also

- [Semantic Modeling Concepts](../explanations/semantic-modeling-concepts.md)
