# OpenAPI Interactive Documentation

The Velum REST API is documented using OpenAPI 3.0 specification.

## Interactive API Documentation

Below is the interactive API documentation powered by ReDoc.

<div id="redoc-container"></div>

<script src="https://cdn.jsdelivr.net/npm/redoc@latest/bundles/redoc.standalone.js"></script>
<script>
  Redoc.init('../openapi.yml', {}, document.getElementById('redoc-container'));
</script>

## OpenAPI Specification Files

- [YAML format](../openapi.yml)
- [Swagger format](../api-docs.yml)

## Postman Collection

A Postman collection is available for testing the API:
[Postman Collection](../.postman/)
