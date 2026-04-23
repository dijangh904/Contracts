# Frontend Integration Guide for Compliance Error Codes

## Overview
This guide provides comprehensive instructions for frontend developers to integrate with the standardized compliance error codes in Vesting-Vault contracts.

## Quick Start

### 1. Error Code Detection
```typescript
// Extract error code from Soroban contract error
function extractErrorCode(error: any): number | null {
  if (error?.message?.includes('contract_error')) {
    const match = error.message.match(/\((\d+)\)/);
    return match ? parseInt(match[1]) : null;
  }
  return null;
}

// Example usage
try {
  await contract.claim(userAddress, vaultId, amount);
} catch (error) {
  const errorCode = extractErrorCode(error);
  if (errorCode) {
    handleComplianceError(errorCode);
  }
}
```

### 2. Error Mapping Implementation
```typescript
// Complete error mapping with user-friendly messages
const COMPLIANCE_ERROR_MAP = {
  400: {
    title: "KYC Verification Required",
    message: "Please complete KYC verification to continue",
    action: "Complete KYC",
    severity: "high",
    route: "/kyc-verification",
    helpLink: "/help/kyc"
  },
  401: {
    title: "KYC Verification Expired",
    message: "Your KYC verification has expired",
    action: "Renew KYC",
    severity: "high", 
    route: "/kyc-renewal",
    helpLink: "/help/kyc-expiry"
  },
  402: {
    title: "Address Restricted",
    message: "Your address is on a restricted list",
    action: "Contact Support",
    severity: "critical",
    route: "/support/compliance",
    helpLink: "/help/restricted-address"
  },
  403: {
    title: "Jurisdiction Not Supported",
    message: "Your jurisdiction is not currently supported",
    action: "Check Supported Regions",
    severity: "high",
    route: "/supported-regions",
    helpLink: "/help/jurisdictions"
  },
  404: {
    title: "Legal Signature Required",
    message: "A valid legal signature is required",
    action: "Provide Signature",
    severity: "medium",
    route: "/legal-signature",
    helpLink: "/help/legal-signature"
  },
  405: {
    title: "Invalid Legal Signature",
    message: "The provided legal signature is invalid",
    action: "Re-submit Signature",
    severity: "medium",
    route: "/legal-signature",
    helpLink: "/help/signature-invalid"
  },
  406: {
    title: "Compliance Check Failed",
    message: "Compliance verification could not be completed",
    action: "Try Again Later",
    severity: "medium",
    route: "/compliance-status",
    helpLink: "/help/compliance-failed"
  },
  407: {
    title: "Amount Exceeds Limit",
    message: "Transaction amount exceeds AML threshold",
    action: "Reduce Amount",
    severity: "high",
    route: "/transaction-limits",
    helpLink: "/help/aml-limits"
  },
  408: {
    title: "High Risk Rating",
    message: "Transaction requires additional verification",
    action: "Complete Verification",
    severity: "medium",
    route: "/risk-verification",
    helpLink: "/help/risk-rating"
  },
  409: {
    title: "Document Verification Failed",
    message: "Required documents could not be verified",
    action: "Re-upload Documents",
    severity: "medium",
    route: "/document-upload",
    helpLink: "/help/document-verification"
  },
  410: {
    title: "Accreditation Required",
    message: "Accredited investor status required",
    action: "Verify Accreditation",
    severity: "high",
    route: "/accreditation",
    helpLink: "/help/accreditation"
  },
  411: {
    title: "Tax Compliance Required",
    message: "Tax information must be provided",
    action: "Complete Tax Forms",
    severity: "medium",
    route: "/tax-compliance",
    helpLink: "/help/tax-compliance"
  },
  412: {
    title: "Regulatory Block Active",
    message: "Transactions are temporarily blocked",
    action: "Wait or Contact Support",
    severity: "high",
    route: "/status",
    helpLink: "/help/regulatory-block"
  },
  413: {
    title: "Whitelist Approval Required",
    message: "Address not on approved whitelist",
    action: "Apply for Whitelist",
    severity: "medium",
    route: "/whitelist-application",
    helpLink: "/help/whitelist"
  },
  414: {
    title: "Blacklist Violation",
    message: "Address is on restricted list",
    action: "Contact Support",
    severity: "critical",
    route: "/support/compliance",
    helpLink: "/help/blacklist"
  },
  415: {
    title: "Geofencing Restriction",
    message: "Access restricted in your location",
    action: "Check Location Settings",
    severity: "medium",
    route: "/location-settings",
    helpLink: "/help/geofencing"
  },
  416: {
    title: "Identity Verification Expired",
    message: "Identity verification has expired",
    action: "Renew Verification",
    severity: "high",
    route: "/identity-verification",
    helpLink: "/help/identity-expiry"
  },
  417: {
    title: "Source of Funds Verification",
    message: "Source of funds must be verified",
    action: "Provide Source Documentation",
    severity: "medium",
    route: "/source-of-funds",
    helpLink: "/help/source-of-funds"
  },
  418: {
    title: "Beneficial Owner Verification",
    message: "Beneficial owner information required",
    action: "Complete Owner Information",
    severity: "medium",
    route: "/beneficial-owner",
    helpLink: "/help/beneficial-owner"
  },
  419: {
    title: "PEP Status Detected",
    message: "Politically exposed person status detected",
    action: "Additional Review Required",
    severity: "high",
    route: "/pep-verification",
    helpLink: "/help/pep-status"
  },
  420: {
    title: "Multiple Sanctions Hits",
    message: "Address appears on multiple sanctions lists",
    action: "Immediate Compliance Review",
    severity: "critical",
    route: "/support/emergency",
    helpLink: "/help/sanctions"
  }
};
```

## React Integration Example

### Component Implementation
```typescript
import React, { useState } from 'react';
import { Alert, Button, Modal } from 'antd';
import { useNavigate } from 'react-router-dom';

interface ComplianceErrorProps {
  error: any;
  onRetry?: () => void;
}

export const ComplianceErrorHandler: React.FC<ComplianceErrorProps> = ({ error, onRetry }) => {
  const [visible, setVisible] = useState(true);
  const navigate = useNavigate();
  
  const errorCode = extractErrorCode(error);
  const errorInfo = errorCode ? COMPLIANCE_ERROR_MAP[errorCode] : null;

  if (!errorInfo) {
    return (
      <Alert
        message="Transaction Failed"
        description="An unknown error occurred. Please try again."
        type="error"
        showIcon
        action={
          onRetry && <Button size="small" onClick={onRetry}>Retry</Button>
        }
      />
    );
  }

  const handleAction = () => {
    if (errorInfo.route) {
      navigate(errorInfo.route);
    }
    setVisible(false);
  };

  return (
    <Modal
      title={errorInfo.title}
      open={visible}
      onCancel={() => setVisible(false)}
      footer={[
        <Button key="cancel" onClick={() => setVisible(false)}>
          Cancel
        </Button>,
        <Button key="action" type="primary" onClick={handleAction}>
          {errorInfo.action}
        </Button>,
        onRetry && (
          <Button key="retry" onClick={onRetry}>
            Retry Transaction
          </Button>
        )
      ]}
    >
      <Alert
        message={errorInfo.message}
        description={
          <div>
            <p>{errorInfo.message}</p>
            <a href={errorInfo.helpLink} target="_blank" rel="noopener noreferrer">
              Learn more about this requirement
            </a>
          </div>
        }
        type={errorInfo.severity === 'critical' ? 'error' : 'warning'}
        showIcon
      />
    </Modal>
  );
};
```

### Hook for Error Handling
```typescript
import { useCallback } from 'react';
import { message } from 'antd';

export const useComplianceErrorHandler = () => {
  const handleError = useCallback((error: any, context?: string) => {
    const errorCode = extractErrorCode(error);
    
    if (errorCode && COMPLIANCE_ERROR_MAP[errorCode]) {
      const errorInfo = COMPLIANCE_ERROR_MAP[errorCode];
      
      // Log error for analytics
      console.error(`Compliance Error ${errorCode} in ${context}:`, error);
      
      // Show user-friendly message
      message.error({
        content: errorInfo.message,
        duration: 5,
        key: `compliance-${errorCode}`
      });
      
      return errorInfo;
    }
    
    // Handle non-compliance errors
    message.error('Transaction failed. Please try again.');
    return null;
  }, []);

  return { handleError };
};
```

## Vue.js Integration Example

### Error Handler Component
```vue
<template>
  <div v-if="errorInfo" class="compliance-error">
    <a-alert
      :type="alertType"
      :message="errorInfo.title"
      :description="errorInfo.message"
      show-icon
    >
      <template #action>
        <a-button @click="handleAction" type="primary">
          {{ errorInfo.action }}
        </a-button>
        <a-button @click="$emit('retry')" v-if="$emit.retry">
          Retry
        </a-button>
      </template>
    </a-alert>
    
    <div class="help-links">
      <a :href="errorInfo.helpLink" target="_blank">
        Learn more about this requirement
      </a>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';

const props = defineProps<{
  error: any;
}>();

const emit = defineEmits<{
  retry: [];
}>();

const router = useRouter();

const errorCode = extractErrorCode(props.error);
const errorInfo = errorCode ? COMPLIANCE_ERROR_MAP[errorCode] : null;

const alertType = computed(() => {
  if (!errorInfo) return 'error';
  switch (errorInfo.severity) {
    case 'critical': return 'error';
    case 'high': return 'warning';
    default: return 'info';
  }
});

const handleAction = () => {
  if (errorInfo?.route) {
    router.push(errorInfo.route);
  }
};
</script>
```

## Angular Integration Example

### Error Service
```typescript
import { Injectable } from '@angular/core';
import { Router } from '@angular/router';
import { BehaviorSubject } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class ComplianceErrorService {
  private errorSubject = new BehaviorSubject<any>(null);
  public currentError$ = this.errorSubject.asObservable();

  constructor(private router: Router) {}

  handleError(error: any, context?: string): void {
    const errorCode = this.extractErrorCode(error);
    const errorInfo = errorCode ? COMPLIANCE_ERROR_MAP[errorCode] : null;

    if (errorInfo) {
      // Log for analytics
      console.error(`Compliance Error ${errorCode} in ${context}:`, error);
      
      // Emit error for components to handle
      this.errorSubject.next({
        ...errorInfo,
        originalError: error,
        context
      });
    } else {
      // Handle non-compliance errors
      this.showGenericError();
    }
  }

  clearError(): void {
    this.errorSubject.next(null);
  }

  private extractErrorCode(error: any): number | null {
    if (error?.message?.includes('contract_error')) {
      const match = error.message.match(/\((\d+)\)/);
      return match ? parseInt(match[1]) : null;
    }
    return null;
  }

  private showGenericError(): void {
    // Show generic error notification
  }
}
```

## Error Recovery Strategies

### 1. Progressive Disclosure
```typescript
const getErrorRecoverySteps = (errorCode: number): string[] => {
  switch (errorCode) {
    case 400: // KYC Not Completed
      return [
        "Visit KYC verification page",
        "Upload required documents",
        "Wait for approval (1-2 business days)",
        "Retry transaction"
      ];
    case 402: // Address Sanctioned
      return [
        "Contact compliance support immediately",
        "Provide additional verification if requested",
        "Wait for compliance review"
      ];
    default:
      return ["Contact support for assistance"];
  }
};
```

### 2. Smart Retry Logic
```typescript
const shouldAllowRetry = (errorCode: number): boolean => {
  // Don't allow retry for critical compliance issues
  const noRetryCodes = [402, 414, 420]; // Sanctions, Blacklist, Multiple sanctions
  return !noRetryCodes.includes(errorCode);
};

const getRetryDelay = (errorCode: number, attemptCount: number): number => {
  // Exponential backoff for temporary issues
  const temporaryIssues = [406, 412]; // Compliance check failed, Regulatory block
  if (temporaryIssues.includes(errorCode)) {
    return Math.min(1000 * Math.pow(2, attemptCount), 30000);
  }
  return 0;
};
```

## Analytics Integration

### Error Tracking
```typescript
interface ComplianceErrorEvent {
  errorCode: number;
  errorTitle: string;
  userId?: string;
  timestamp: string;
  context: string;
  userAgent: string;
  resolved: boolean;
}

export const trackComplianceError = (errorInfo: any, context: string): void => {
  const event: ComplianceErrorEvent = {
    errorCode: errorInfo.errorCode,
    errorTitle: errorInfo.title,
    userId: getCurrentUserId(),
    timestamp: new Date().toISOString(),
    context,
    userAgent: navigator.userAgent,
    resolved: false
  };

  // Send to analytics service
  analytics.track('compliance_error', event);
};
```

### Success Metrics
```typescript
export const trackComplianceResolution = (errorCode: number, resolutionType: string): void => {
  analytics.track('compliance_resolved', {
    errorCode,
    resolutionType, // 'user_action', 'automatic', 'support_intervention'
    timestamp: new Date().toISOString()
  });
};
```

## Testing Strategy

### Unit Tests
```typescript
describe('Compliance Error Handling', () => {
  test('should extract error code from contract error', () => {
    const error = { message: 'contract_error: KycNotCompleted(400)' };
    expect(extractErrorCode(error)).toBe(400);
  });

  test('should return correct error mapping for KYC error', () => {
    const errorInfo = COMPLIANCE_ERROR_MAP[400];
    expect(errorInfo.title).toBe('KYC Verification Required');
    expect(errorInfo.action).toBe('Complete KYC');
    expect(errorInfo.severity).toBe('high');
  });

  test('should handle unknown error gracefully', () => {
    const error = { message: 'unknown error' };
    expect(extractErrorCode(error)).toBeNull();
  });
});
```

### Integration Tests
```typescript
describe('Compliance Error Integration', () => {
  test('should show KYC modal when KYC error occurs', async () => {
    const mockError = { message: 'contract_error: KycNotCompleted(400)' };
    
    render(<ComplianceErrorHandler error={mockError} />);
    
    expect(screen.getByText('KYC Verification Required')).toBeInTheDocument();
    expect(screen.getByText('Complete KYC')).toBeInTheDocument();
  });

  test('should navigate to correct route on action click', async () => {
    const mockError = { message: 'contract_error: KycNotCompleted(400)' };
    const mockNavigate = jest.fn();
    
    render(<ComplianceErrorHandler error={mockError} />, { 
      wrapper: ({ children }) => <Router>{children}</Router> 
    });
    
    fireEvent.click(screen.getByText('Complete KYC'));
    expect(mockNavigate).toHaveBeenCalledWith('/kyc-verification');
  });
});
```

## Best Practices

### 1. User Experience
- Always show user-friendly messages, never raw error codes
- Provide clear next steps for each error type
- Use appropriate severity levels for UI feedback
- Include help links for additional information

### 2. Performance
- Cache error mappings to avoid repeated lookups
- Use lazy loading for help content
- Implement error boundaries to prevent crashes

### 3. Accessibility
- Ensure error messages are screen-reader friendly
- Provide keyboard navigation for error modals
- Use appropriate ARIA labels and roles

### 4. Security
- Never expose sensitive information in error messages
- Sanitize user inputs when displaying in error contexts
- Implement rate limiting for error-prone operations

### 5. Monitoring
- Track all compliance errors for regulatory reporting
- Monitor error resolution rates
- Set up alerts for critical compliance issues

This comprehensive integration guide ensures frontend applications can effectively handle and display compliance errors, providing users with clear guidance and improving overall user experience.
