# Compliance Error Codes Mapping Guide

## Overview
This document provides a comprehensive mapping of standardized compliance error codes for frontend integration with Vesting-Vault contracts.

## Error Code Structure

### Format: `Error::ErrorCodeName (NumericCode)`

### Compliance Error Range: 400-420

| Error Code | Numeric Value | Category | User Message | Suggested Action |
|------------|---------------|----------|--------------|------------------|
| `KycNotCompleted` | 400 | KYC | "KYC verification required" | Redirect to KYC verification flow |
| `KycExpired` | 401 | KYC | "KYC verification expired" | Prompt user to renew KYC |
| `AddressSanctioned` | 402 | Sanctions | "Address is on sanctions list" | Contact compliance support |
| `JurisdictionRestricted` | 403 | Geographic | "Jurisdiction not supported" | Show supported jurisdictions |
| `LegalSignatureMissing` | 404 | Legal | "Legal signature required" | Guide to signature process |
| `LegalSignatureInvalid` | 405 | Legal | "Legal signature is invalid" | Re-submit signature |
| `ComplianceCheckFailed` | 406 | General | "Compliance verification failed" | Contact support |
| `AmlThresholdExceeded` | 407 | AML | "Amount exceeds AML threshold" | Reduce amount or verify source |
| `RiskRatingTooHigh` | 408 | Risk | "Risk rating too high" | Additional verification needed |
| `DocumentVerificationFailed` | 409 | Documents | "Document verification failed" | Re-upload documents |
| `AccreditationStatusInvalid` | 410 | Accreditation | "Accreditation status invalid" | Verify investor status |
| `TaxComplianceFailed` | 411 | Tax | "Tax compliance required" | Complete tax forms |
| `RegulatoryBlockActive` | 412 | Regulatory | "Regulatory block active" | Wait or contact support |
| `WhitelistNotApproved` | 413 | Access Control | "Not on approved whitelist" | Apply for whitelist approval |
| `BlacklistViolation` | 414 | Access Control | "Address is blacklisted" | Contact compliance team |
| `GeofencingRestriction` | 415 | Geographic | "Geofencing restriction active" | Check location settings |
| `IdentityVerificationExpired` | 416 | Identity | "Identity verification expired" | Renew identity verification |
| `SourceOfFundsNotVerified` | 417 | AML | "Source of funds not verified" | Provide source documentation |
| `BeneficialOwnerNotVerified` | 418 | AML | "Beneficial owner not verified" | Complete beneficial owner form |
| `PoliticallyExposedPerson` | 419 | PEP | "PEP status detected" | Additional compliance review |
| `SanctionsListHit` | 420 | Sanctions | "Multiple sanctions list hits" | Immediate compliance review |

## Frontend Integration Examples

### React/TypeScript Example

```typescript
enum ComplianceError {
  KycNotCompleted = 400,
  KycExpired = 401,
  AddressSanctioned = 402,
  // ... other error codes
}

interface ClaimResponse {
  success: boolean;
  error?: {
    code: number;
    message: string;
    action: string;
  };
}

async function handleClaim(vestingId: number, amount: bigint): Promise<ClaimResponse> {
  try {
    await contract.claim(userAddress, vestingId, amount);
    return { success: true };
  } catch (error) {
    const errorCode = extractErrorCode(error);
    const errorMapping = getErrorMapping(errorCode);
    
    return {
      success: false,
      error: {
        code: errorCode,
        message: errorMapping.message,
        action: errorMapping.action
      }
    };
  }
}

function getErrorMapping(code: number) {
  switch (code) {
    case ComplianceError.KycNotCompleted:
      return {
        message: "KYC verification required",
        action: "redirect_to_kyc"
      };
    case ComplianceError.AddressSanctioned:
      return {
        message: "Address is on sanctions list",
        action: "contact_support"
      };
    // ... handle all other cases
    default:
      return {
        message: "Unknown error occurred",
        action: "contact_support"
      };
  }
}
```

### Vue.js Example

```javascript
const complianceErrorMessages = {
  400: {
    title: "KYC Required",
    message: "Please complete KYC verification to continue",
    action: "Complete KYC",
    route: "/kyc-verification"
  },
  402: {
    title: "Compliance Issue",
    message: "Your address appears on restricted lists",
    action: "Contact Support",
    route: "/support/compliance"
  },
  // ... other mappings
};

export function handleClaimError(errorCode) {
  const error = complianceErrorMessages[errorCode];
  if (error) {
    showErrorDialog(error);
    if (error.route) {
      router.push(error.route);
    }
  }
}
```

## Error Response Format

### Soroban Contract Error Response
```json
{
  "error": {
    "code": 400,
    "message": "KycNotCompleted",
    "type": "contract_error"
  }
}
```

### Frontend Parsed Response
```json
{
  "success": false,
  "error": {
    "code": 400,
    "title": "KYC Required",
    "message": "KYC verification required",
    "action": "redirect_to_kyc",
    "userFriendly": "Please complete KYC verification to claim tokens",
    "nextSteps": [
      "Visit KYC verification page",
      "Upload required documents",
      "Wait for approval (usually 1-2 business days)"
    ]
  }
}
```

## Implementation Guidelines

### 1. Error Code Extraction
```javascript
function extractContractError(error) {
  // Parse Soroban error format
  if (error.message && error.message.includes("contract_error")) {
    const match = error.message.match(/\((\d+)\)/);
    return match ? parseInt(match[1]) : null;
  }
  return null;
}
```

### 2. Progressive Enhancement
- Start with basic error messages
- Add user-friendly explanations
- Include actionable next steps
- Provide contextual help links

### 3. Localization Support
```javascript
const errorMessages = {
  en: {
    400: "KYC verification required",
    402: "Address is on sanctions list"
  },
  es: {
    400: "Verificación KYC requerida",
    402: "La dirección está en listas de sanciones"
  }
};
```

## Testing Strategy

### Unit Tests
```javascript
describe('Compliance Error Handling', () => {
  test('should handle KYC not completed error', () => {
    const error = { code: 400 };
    const result = handleClaimError(error);
    expect(result.action).toBe('redirect_to_kyc');
  });
  
  test('should handle sanctions error', () => {
    const error = { code: 402 };
    const result = handleClaimError(error);
    expect(result.action).toBe('contact_support');
  });
});
```

### Integration Tests
- Test actual contract interactions
- Verify error code propagation
- Test frontend error handling flow

## Support Integration

### Help Desk Integration
```javascript
function createSupportTicket(errorCode, userContext) {
  const ticket = {
    subject: `Compliance Error ${errorCode}`,
    category: getErrorCategory(errorCode),
    priority: getErrorPriority(errorCode),
    userContext,
    errorDetails: getErrorDetails(errorCode)
  };
  
  return supportAPI.createTicket(ticket);
}
```

### Analytics Integration
```javascript
function trackComplianceError(errorCode, userId) {
  analytics.track('compliance_error', {
    error_code: errorCode,
    error_category: getErrorCategory(errorCode),
    user_id: userId,
    timestamp: new Date().toISOString()
  });
}
```

## Best Practices

1. **Always show user-friendly messages** - Never display raw error codes to users
2. **Provide clear next steps** - Each error should have an actionable resolution
3. **Maintain consistency** - Use the same error handling pattern across all components
4. **Log errors for analysis** - Track compliance errors for regulatory reporting
5. **Test edge cases** - Verify error handling works for all compliance scenarios
6. **Update documentation** - Keep error mapping docs in sync with contract changes
