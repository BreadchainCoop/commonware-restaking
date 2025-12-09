{{/*
Expand the name of the chart.
*/}}
{{- define "commonware-avs.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "commonware-avs.fullname" -}}
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
{{- define "commonware-avs.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "commonware-avs.labels" -}}
helm.sh/chart: {{ include "commonware-avs.chart" . }}
{{ include "commonware-avs.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "commonware-avs.selectorLabels" -}}
app.kubernetes.io/name: {{ include "commonware-avs.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Ethereum service name
*/}}
{{- define "commonware-avs.ethereum.fullname" -}}
{{- printf "%s-ethereum" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Signer service name
*/}}
{{- define "commonware-avs.signer.fullname" -}}
{{- printf "%s-signer" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Router service name
*/}}
{{- define "commonware-avs.router.fullname" -}}
{{- printf "%s-router" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Node name helper
*/}}
{{- define "commonware-avs.node.fullname" -}}
{{- printf "%s-node" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Setup job name
*/}}
{{- define "commonware-avs.setup.fullname" -}}
{{- printf "%s-setup" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Shared data PVC name
*/}}
{{- define "commonware-avs.shareddata.fullname" -}}
{{- printf "%s-shared-data" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Config ConfigMap name
*/}}
{{- define "commonware-avs.config.fullname" -}}
{{- printf "%s-config" (include "commonware-avs.fullname" .) }}
{{- end }}

{{/*
Secret name - supports existing secret or creates new one
*/}}
{{- define "commonware-avs.secret.fullname" -}}
{{- if .Values.secrets.existingSecret }}
{{- .Values.secrets.existingSecret }}
{{- else }}
{{- printf "%s-secret" (include "commonware-avs.fullname" .) }}
{{- end }}
{{- end }}