{{/*
Expand the name of the chart.
*/}}
{{- define "cim-workflow.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "cim-workflow.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "cim-workflow.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "cim-workflow.labels" -}}
helm.sh/chart: {{ include "cim-workflow.chart" . }}
{{ include "cim-workflow.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: cim-domain-workflow
app.kubernetes.io/component: workflow-engine
{{- end }}

{{/*
Selector labels
*/}}
{{- define "cim-workflow.selectorLabels" -}}
app.kubernetes.io/name: {{ include "cim-workflow.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "cim-workflow.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "cim-workflow.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Generate database connection string
*/}}
{{- define "cim-workflow.databaseUrl" -}}
{{- if .Values.postgresql.enabled }}
postgresql://{{ .Values.postgresql.auth.username }}:{{ .Values.postgresql.auth.password }}@{{ include "cim-workflow.fullname" . }}-postgresql:5432/{{ .Values.postgresql.auth.database }}
{{- else }}
{{- .Values.postgresql.external.url }}
{{- end }}
{{- end }}

{{/*
Generate NATS connection string
*/}}
{{- define "cim-workflow.natsUrl" -}}
{{- if .Values.nats.enabled }}
nats://{{ include "cim-workflow.fullname" . }}-nats:4222
{{- else }}
{{- .Values.nats.external.url }}
{{- end }}
{{- end }}

{{/*
Create the name of the PVC to use for persistence
*/}}
{{- define "cim-workflow.persistentVolumeClaimName" -}}
{{- if .Values.persistence.existingClaim }}
{{- .Values.persistence.existingClaim }}
{{- else }}
{{- include "cim-workflow.fullname" . }}-data
{{- end }}
{{- end }}

{{/*
Generate common environment variables
*/}}
{{- define "cim-workflow.commonEnvVars" -}}
- name: RUST_LOG
  value: {{ .Values.config.logLevel | quote }}
- name: ENVIRONMENT
  value: {{ .Values.config.environment | quote }}
- name: POD_NAME
  valueFrom:
    fieldRef:
      fieldPath: metadata.name
- name: POD_NAMESPACE
  valueFrom:
    fieldRef:
      fieldPath: metadata.namespace
- name: NODE_NAME
  valueFrom:
    fieldRef:
      fieldPath: spec.nodeName
{{- end }}